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

def discover_coils(client, config):
    all_coil_addresses = set(config["output_module"]["coil_addresses"].values())
    all_coil_addresses.update(config["input_module"]["coil_addresses"].values())
    
    existing_coil_addresses = set()
    
    try:
        for address in all_coil_addresses:
            result = client.read_coils(address, 1)
            if result.bits:
                existing_coil_addresses.add(address)
    except ModbusIOException:
        pass
    
    missing_coil_addresses = all_coil_addresses - existing_coil_addresses
    
    return existing_coil_addresses, missing_coil_addresses

def discover_coils_registers(client):
    try:
        # Read coils and registers
        coils = client.read_coils(0, 16)  # Read first 16 coils
        registers = client.read_holding_registers(0, 16)  # Read first 16 registers
        return coils.bits, registers.registers
    except ModbusIOException:
        print("Failed to read coils and registers.")
        return None, None

def read_coil(client, address):
    try:
        result = client.read_coils(address,1)
        if isinstance(result,ModbusIOException):
            raise result
        return result.bits[0]
    except ModbusIOException:
        print("Failed to read coil at address", address)
        return None

def write_coil(client, address, value):
    try:
        client.write_coil(address, value)
        print("Coil at address", address, "set to", value)
    except ModbusIOException:
        print("Failed to write coil at address", address)

def read_register(client, address):
    try:
        result = client.read_holding_registers(address, 1)
        
        return result.registers[0]
    except ModbusIOException:
        print("Failed to read register at address", address)
        return None

def write_register(client, address, value):
    try:
        client.write_register(address, value)
        print("Register at address", address, "set to", value)
    except ModbusIOException:
        print("Failed to write register at address", address)

def read_output_status(client, config, output_name):
    output_address = config["output_module"]["coil_addresses"].get(output_name)
    
    if output_address is not None:
        result = client.read_coils(output_address, 1)
        if result.bits:
            return result.bits[0]
    
    return None

def run_example_1():
    client = ModbusTcpClient(PLC_IP, port=MODBUS_PORT)
    if client.connect():
        coils, registers = discover_coils_registers(client)

        if coils is not None and registers is not None:
            print("Discovered Coils:", coils)
            print("Discovered Registers:", registers)
            
            # Example usage
            coil_address = 0
            register_address = 0
            
            read_value = read_coil(client, coil_address)
            if read_value is not None:
                print("Read Coil Value:", read_value)
            
            write_coil(client, coil_address, True)
            
            read_value = read_register(client, register_address)
            if read_value is not None:
                print("Read Register Value:", read_value)
            
            write_register(client, register_address, 123)
            
        client.close()
    else:
        print("Failed to connect to the PLC.")

def run_example_2():
    config = load_config("config.json")
    client = ModbusTcpClient(PLC_IP, port=MODBUS_PORT)
    
    if client.connect():
        # Replace 'output_1' with the name of the desired output
        output_name = 'output_1'
        
        led_status = read_output_status(client, config, output_name)
        if led_status is not None:
            print("LED Status:", led_status)
            
            # Toggle LED based on status
            if led_status:
                print("Turning LED OFF")
                write_coil(client, config["output_module"]["coil_addresses"][output_name], False)
            else:
                print("Turning LED ON")
                write_coil(client, config["output_module"]["coil_addresses"][output_name], True)
        
        client.close()
    else:
        print("Failed to connect to the PLC.")

if __name__ == "__main__":
    # run_example_1()
    run_example_2()
