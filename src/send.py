#!/usr/bin/python3

import sys
import socket
import itertools

addr = 'localhost'
port = 8181

def separate(bytes, window=1):
	return ' '.join([ bytes[i:i+window].hex() for i in range(0, len(bytes), window) ])

def parse_rsp_get(bs):
	val_len = int(bs.__next__())
	val = bytes(itertools.islice(bs, val_len))
	# val = bs[1:val_len+1]
	print(val.decode())
	# return bs[val_len+1:]

def parse_rsp_put(bs):
	b = bs.__next__()
	print('OK' if b == 0 else 'ERR {}'.format(b))
	# return bs[1:]

def send(bs):
	print('sent: {}'.format(separate(bs)))

	s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
	s.connect((addr, port))
	s.send(bs)

	s.settimeout(1)
	while True:
		try:
			for b in s.recv(128):
				yield b
		except:
			break;
	s.close()

def raw(args):
	return bytes.fromhex(''.join(args))

def get(args):
	key = args[0]
	keybytes = key.encode()
	size = len(keybytes)
	msb = size // 256
	lsb = size % 256
	pkg = bytes([msb, lsb, 0]) + keybytes
	return pkg, parse_rsp_get

def put(args):
	key = args[0]
	val = args[1]
	keybytes = key.encode()
	valbytes = val.encode()
	size = 1 + len(keybytes) + len(valbytes)
	msb = size // 256
	lsb = size % 256
	pkg = bytes([msb, lsb, 1, len(keybytes)]) + keybytes + valbytes
	return pkg, parse_rsp_put

args = sys.argv[1:]
to_send = b''
to_parse = []

while len(args) > 0:
	subcmd = args[0].lower()

	if subcmd == 'raw':
		to_send += raw(args[1:2])
		args = args[2:]
	elif subcmd == 'get':
		to_send_e, to_parse_e = get(args[1:2])
		to_send += to_send_e
		to_parse += [to_parse_e]
		args = args[2:]
	elif subcmd == 'put':
		to_send_e, to_parse_e = put(args[1:3])
		to_send += to_send_e
		to_parse += [to_parse_e]
		args = args[3:]
	else:
		print('invalid subcommand')

rsp = send(to_send)
for parser in to_parse:
	parser(rsp)


# for arg in sys.argv[1:]:
# 	val = int(arg, 16);