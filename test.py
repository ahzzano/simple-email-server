import socket

host = "127.0.0.1"
port = 2525


def get_from_sock(sock):
    buf = b""
    while not buf.endswith(b"\r\n"):
        buf += sock.recv(1)

    return buf.decode().strip()


with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as sock:
    sock.connect((host, port))

    line = get_from_sock(sock).replace("\r\n", "")

    print(f"S: {line}")

    if not line.startswith("220"):
        print("ERROR: Line didn't start with 220")

    sock.send(b"HELO crepes.fr\r\n")
    line = get_from_sock(sock)
    print(line)

    sock.send(b"MAIL FROM:<bob@example.org>\r\n")
    line = get_from_sock(sock)
    print(line)

    sock.send(b"RCPT TO:<alice@example.com>\r\n")
    line = get_from_sock(sock)
    print(line)

    sock.send(b"DATA\r\n")
    line = get_from_sock(sock)
    print(line)

    print("=====DATA=====")

    print("I love France")
    sock.send(b"I love France")
    sock.send(b"\r\n")
    line = get_from_sock(sock)
    print(line)

    print("DONE")
