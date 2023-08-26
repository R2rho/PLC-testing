from pymodbus.client import ModbusTcpClient
from pymodbus.exceptions import ModbusIOException
import json

# PLC IP address
PLC_IP = "192.168.20.50"
# Modbus port
MODBUS_PORT = 502

def load_config(filename):
    with open(filename, "r") as file:
        config = json.load(file)
    return config

def discover_modules(client, config):
    discovered_modules = []
    
    for module_config in config["modules"]:
        start_address = module_config["start_address"]
        end_address = module_config["end_address"]
        
        existing_addresses = set()
        
        try:
            for address in range(start_address, end_address + 1):
                result = client.read_coils(address, 1)
                if result.bits:
                    existing_addresses.add(address)
        except ModbusIOException:
            pass
        
        if existing_addresses:
            discovered_modules.append((module_config["name"], existing_addresses))
    
    return discovered_modules

if __name__ == "__main__":
    config = load_config("modules.json")
    client = ModbusTcpClient(PLC_IP, port=MODBUS_PORT)
    
    if client.connect():
        discovered_modules = discover_modules(client, config)
        
        for module_name, addresses in discovered_modules:
            print("Discovered", module_name, "at addresses:", addresses)
        
        # ... (rest of the program)
        
        client.close()
    else:
        print("Failed to connect to the PLC.")
