#!/usr/bin/python3

import sys
import socket

addr = 'localhost'
port = 8181

def separate(bytes, window=1):
	return ' '.join([ bytes[i:i+window].hex() for i in range(0, len(bytes), window) ])

def send(bs):
	print('sent: {}'.format(separate(bs)))

	s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
	s.connect((addr, port))
	s.send(bs)

	s.settimeout(1)
	try:
		rsp = s.recv(128)
		print('rcvd: {}'.format(separate(rsp)))
	except:
		print('timeout')

	s.close()


subcmd = sys.argv[1].lower()

if subcmd == 'raw':
	bs = bytes.fromhex(''.join(sys.argv[2:]))
	send(bs)
elif subcmd == 'get':
	key = sys.argv[2]
	keybytes = key.encode()
	size = len(keybytes)
	msb = size // 256
	lsb = size % 256
	pkg = bytes([msb, lsb, 0]) + keybytes
	send(pkg)
elif subcmd == 'put':
	key = sys.argv[2]
	val = sys.argv[3]
	keybytes = key.encode()
	valbytes = val.encode()
	size = 1 + len(keybytes) + len(valbytes)
	msb = size // 256
	lsb = size % 256
	pkg = bytes([msb, lsb, 1, len(keybytes)]) + keybytes + valbytes
	send(pkg)
else:
	print('invalid subcommand')

# for arg in sys.argv[1:]:
# 	val = int(arg, 16);