from pyModbusTCP.client import ModbusClient

SERVER_HOST = "192.168.20.50"
PORT = 502
CW_COIL = 0  # Cool White coil address
WW_COIL = 1  # Warm White coil address
UNIT = 0x01

#Create an instance of ModbusServer
client = ModbusClient(host=SERVER_HOST,port=PORT, unit_id=1, auto_open=True)

try:
    print('Connecting to server')
    c = client.read_coils(0,20)
    print("Connection successful")

    if c:
        print(c)
except:
    pass