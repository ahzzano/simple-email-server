import socket 

host = "127.0.0.1" 
port = 2525

with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    sock.connect((host, port))

    buf = b""
    while not buf.endswith(b"\r\n"):
        buf += sock.recv(1)
    
    line = buf.decode().strip()
    print(f"S: {line}")

    if not line.startswith("220"):
        print("ERROR: Line didn't start with 220")
    
    sock.send(b"HELO crepes.fr")

