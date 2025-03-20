connect from_socket_file="/tmp/source.sock":
    socat - UNIX-SENDTO:skean_tic_tac_toe.sock,bind="{{ from_socket_file }}"