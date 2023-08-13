use tokio::runtime::Runtime;
use tokio_modbus::prelude::*;
use tokio_modbus::client::Context;
use std::io::Error;
use std::net::SocketAddr;
use tokio::time::Duration;
use tokio::time::sleep;
enum State {
    On,
    Off,
    Toggle,
}

const SERVER_HOST: &str = "192.168.20.50:502"; // replace with your actual port number
const CW_COIL: u16 = 0;  // Cool White coil address
const WW_COIL: u16 = 1;  // Warm White coil address

async fn control_coil(context: &mut Context, coil: u16, desired_state: State) -> Result<(), Error> {
    let current_state = context.read_coils(coil, 10).await?;
    // println!("Current state: {:?}", current_state);

    match desired_state {
        State::Off => {
            context.write_single_coil(coil, true).await?;
        },
        State::On => {
            context.write_single_coil(coil, false).await?;
        },
        State::Toggle => {
            context.write_single_coil(coil, !current_state[0]).await?;
        },
    }
    // let new_state = context.read_coils(coil, 1).await?;
    // println!("New state: {:?}, NOT current_state: {}", new_state, !current_state[0]);
    Ok(())
}

async fn cycle_leds(context: &mut Context) -> Result<(), Error> {
    // Turn on CW LED and wait for 2 seconds
    let addr: SocketAddr = SERVER_HOST.parse().unwrap();
    *context = tcp::connect(addr).await?;
    println!("COOL WHITE");
    control_coil(context, CW_COIL, State::On).await?;
    sleep(Duration::from_secs(2)).await;
    control_coil(context, CW_COIL, State::Off).await?;
    

    // Turn on WW LED and wait for 2 seconds
    println!("WARM WHITE");
    control_coil(context, WW_COIL, State::On).await?;
    sleep(Duration::from_secs(2)).await;
    control_coil(context, WW_COIL, State::Off).await?;
    context.disconnect().await?;
    Ok(())
}



fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = Runtime::new()?;
    let addr: SocketAddr = SERVER_HOST.parse()?;

    rt.block_on(async {
        loop {
            let mut context = tcp::connect(addr).await?;
            println!("Enter CW (Cool White), WW (Warm White), CYCLE, or Q (Quit): ");
            let mut led_to_toggle = String::new();
            std::io::stdin().read_line(&mut led_to_toggle)?;
            let led_to_toggle = led_to_toggle.trim().to_lowercase();

            match led_to_toggle.as_str() {
                "cwon" => {
                    control_coil(&mut context, CW_COIL, State::On).await?;
                }
                "cwoff" => {
                    control_coil(&mut context, CW_COIL, State::Off).await?;
                }
                "cw" => {
                    control_coil(&mut context, CW_COIL, State::Toggle).await?;
                }
                "wwon" => {
                    control_coil(&mut context, WW_COIL, State::On).await?;
                }
                "wwoff" => {
                    control_coil(&mut context, WW_COIL, State::Off).await?;
                }
                "ww" => {
                    control_coil(&mut context, WW_COIL, State::Toggle).await?;
                }
                "cycle" => {
                    loop{
                        cycle_leds(&mut context).await?;

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