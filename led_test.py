from pymodbus.client import ModbusTcpClient
from enum import Enum

class State(Enum):
    ON = 1
    OFF = 2
    TOGGLE = 3

SERVER_HOST = "192.168.20.50"
PORT = 502
CW_COIL = 0  # Cool White coil address
WW_COIL = 1  # Warm White coil address
UNIT = 0x01

def control_coil(client: ModbusTcpClient, coil: int, desired_state: State) -> None:
    # Read the current state of the coil
    # current_state = client.read_coils(coil, 1).bits[0]
    current_state = client.read_coils(address=coil,count=1,units=UNIT).bits[0]

    if desired_state == State.ON:
        # Turn on the coil
        client.write_coil(coil, True)
    elif desired_state == State.OFF:
        # Turn off the coil
        client.write_coil(coil, False)
    elif desired_state == State.TOGGLE:
        # Toggle the coil
        client.write_coil(coil, not current_state)

if __name__ == "__main__":
    client = ModbusTcpClient(SERVER_HOST, port=PORT)
    client.connect()

    while True:
        #reconnect if connection lost
        if not client.connected: client.connect()
        led_to_toggle = input("Enter CW (Cool White), WW (Warm White), or Q (Quit):    ").lower()
        # print(f'LED to toggle: {led_to_toggle}')
        if client.connected:    
            if led_to_toggle == 'cw':
                control_coil(client=client, coil=CW_COIL, desired_state=State.TOGGLE)
            elif led_to_toggle == 'ww':
                control_coil(client=client, coil=WW_COIL, desired_state=State.TOGGLE)
            else:
                raise Exception(f'No LED called "{led_to_toggle}"')
        else:
            print(f'Lost connection with TCP client at {SERVER_HOST}')
            break
