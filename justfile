connect from_socket_file="/tmp/source.sock":
    socat - UNIX-SENDTO:sk_xo.sock,bind="{{ from_socket_file }}"