/*
    Copyright (c) 2007-2016 Contributors as noted in the AUTHORS file

    This file is part of libzmq, the ZeroMQ core engine in C++.

    libzmq is free software; you can redistribute it and/or modify it under
    the terms of the GNU Lesser General Public License (LGPL) as published
    by the Free Software Foundation; either version 3 of the License, or
    (at your option) any later version.

    As a special exception, the Contributors give you permission to link
    this library with independent modules to produce an executable,
    regardless of the license terms of these independent modules, and to
    copy and distribute the resulting executable under terms of your choice,
    provided that you also meet, for each linked independent module, the
    terms and conditions of the license of that module. An independent
    module is a module which is not derived from or based on this library.
    If you modify this library, you must extend this exception to your
    version of the library.

    libzmq is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or
    FITNESS FOR A PARTICULAR PURPOSE. See the GNU Lesser General Public
    License for more details.

    You should have received a copy of the GNU Lesser General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/

#include <zmq.h>
#include <stdio.h>
#include <stdlib.h>
#include <unistd.h>

int main(int argc, char *argv[])
{
    char *payload_value = NULL;
    const char *peer = NULL;
    void *ctx = NULL;
    void *s = NULL;
    int rc = 0;
    int c = 0;
    int payload = 0;
    zmq_msg_t msg;

    // Parsing arguments
    while ((c = getopt(argc, argv, ":e:p:")) != -1)
    {
        switch (c)
        {
        case 'p':
            payload_value = optarg;
            break;
        case 'e':
            peer = optarg;
            break;
        default:
            break;
        }
    }

    if (peer == NULL || payload_value == NULL)
    {
        printf("Usage:\n\t./zmq_pub_thr -e tcp://127.0.0.1:4505 -p 8\n");
        exit(EXIT_FAILURE);
    }
    payload = atoi(payload_value);

    ctx = zmq_init(1);
    if (!ctx)
    {
        printf("error in zmq_init: %s\n", zmq_strerror(errno));
        return -1;
    }

    s = zmq_socket(ctx, ZMQ_PUSH);
    if (!s)
    {
        printf("error in zmq_socket: %s\n", zmq_strerror(errno));
        return -1;
    }

    rc = zmq_connect(s, peer);
    if (rc != 0)
    {
        printf("error in zmq_connect: %s\n", zmq_strerror(errno));
        return -1;
    }

    while (1)
    {
        rc = zmq_msg_init_size(&msg, payload);
        if (rc != 0)
        {
            printf("error in zmq_msg_init_size: %s\n", zmq_strerror(errno));
            return -1;
        }
        rc = zmq_sendmsg(s, &msg, 0);
        if (rc < 0)
        {
            printf("error in zmq_sendmsg: %s\n", zmq_strerror(errno));
            return -1;
        }
        rc = zmq_msg_close(&msg);
        if (rc != 0)
        {
            printf("error in zmq_msg_close: %s\n", zmq_strerror(errno));
            return -1;
        }
    }

    return 0;
}