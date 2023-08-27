use tokio::runtime::Runtime;
use tokio_modbus::prelude::*;
use tokio_modbus::client::Context;
use std::io::Error;
use std::net::SocketAddr;
use std::time::Instant;
use tokio::time::Duration;
use tokio::time::sleep;
use tokio::sync::RwLock;
use std::sync::Arc;
enum State {
    On,
    Off,
    Toggle,
}

struct LEDDelay{
    delay: RwLock<f64>
}

impl LEDDelay {
    fn new() -> Self {
        LEDDelay { delay: RwLock::new(0.75) } //default delay
    }

    async fn set(&self, value: f64){
        *self.delay.write().await = value
    }

    async fn get(&self) -> f64 {
        *self.delay.read().await
    }
}

async fn check_light_curtain(context: &mut Context, delay_state: Arc<LEDDelay>) -> Result<(), Error> {
    loop {
        let light_curtain_crossed = context.read_discrete_inputs(LIGHT_CURTAIN, 1).await?[0];
        if light_curtain_crossed {
            delay_state.set(0.05).await;
        }
        else{
            delay_state.set(0.75).await;
        }
        sleep(Duration::from_millis(100)).await; //Check every 100ms
    }
}

async fn cycle_leds_continuous(context: &mut Context, delay_state: Arc<LEDDelay>) -> Result<(), Error> {
    //SET the initial states
    control_coil(context, CW_COIL, State::On).await?;
    control_coil(context, WW_COIL, State::Off).await?;

    let mut current_coil = CW_COIL;

    //CREATE a timer
    let mut last_toggle_time = Instant::now();
    loop {
        
        //CHECK the time delay status
        let current_time = Instant::now();
        let elapsed = current_time.duration_since(last_toggle_time);
        let delay = delay_state.get().await;
        
        if elapsed.as_secs_f64() >= delay {
            //FLIP the coil being toggled
            if current_coil == CW_COIL {
                current_coil = WW_COIL;
                println!("WARM WHITE");
            }
            else {
                current_coil = CW_COIL;
                println!("COOL WHITE");
            }

            //TOGGLE the coil
            control_coil(context, current_coil, State::Toggle).await?;    
            
            //RESTART the delay
            last_toggle_time = current_time;
        }
        
        //Short sleep to prevent busy-waiting and eating CPU time
        sleep(Duration::from_millis(5)).await;

    }
}

const SERVER_HOST: &str = "192.168.20.50:502"; // replace with your actual port number
const CW_COIL: u16 = 512;  // Cool White coil address
const WW_COIL: u16 = 513;  // Warm White coil address
const LIGHT_CURTAIN: u16 = 0; //Digital Input address of the light curtain sensor

async fn control_coil(context: &mut Context, coil: u16, desired_state: State) -> Result<(), Error> {
    let current_state = context.read_coils(coil, 10).await?;
    
    let new_state: bool = match desired_state {
        State::Off => {
            true //LED PLC digital output requires 24V for Off state
        },
        State::On => {
            false //LED PLC digital output requires 0V for On state
        },
        State::Toggle => {
            if current_state[0] {false} else {true}
        },
    };
    
    context.write_single_coil(coil, new_state).await?;
    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = Runtime::new()?;
    let addr: SocketAddr = SERVER_HOST.parse()?;

    rt.block_on(async {

        let shared_delay = Arc::new(LEDDelay::new());

        //Create a clone of the delay state for the light curtain check loop
        let curtain_check_delay = shared_delay.clone();

        //Sparn the continuous light curtain checking loop
        tokio::spawn(async move {
            let mut context = tcp::connect(addr).await.unwrap();
            check_light_curtain(&mut context, curtain_check_delay).await.unwrap();
        });

        loop {
            let mut context = tcp::connect(addr).await?;
            println!("Enter CW (Cool White), WW (Warm White), ON, OFF, CYCLE, or Q (Quit): ");
            let mut led_to_toggle = String::new();
            std::io::stdin().read_line(&mut led_to_toggle)?;
            let led_to_toggle = led_to_toggle.trim().to_lowercase();

            match led_to_toggle.as_str() {
                "cw" => {
                    control_coil(&mut context, CW_COIL, State::Toggle).await?;
                }
                "ww" => {
                    control_coil(&mut context, WW_COIL, State::Toggle).await?;
                }
                "on" => {
                    control_coil(&mut context, CW_COIL, State::On).await?;
                    control_coil(&mut context, WW_COIL, State::On).await?;
                }
                "off" => {
                    control_coil(&mut context, CW_COIL, State::Off).await?;
                    control_coil(&mut context, WW_COIL, State::Off).await?;
                }
                "cycle" => {
                    loop{
                        // cycle_leds(&mut context).await?;
                        cycle_leds_continuous(&mut context, shared_delay.clone()).await?;
                    }
                }
                "q" => break,
                _ => {
                    println!("No LED called {:?}", led_to_toggle);
                }
            }
            context.disconnect().await?;
        }
        Ok(())
    })
}
