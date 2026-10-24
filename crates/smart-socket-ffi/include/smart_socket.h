/* Умная розетка с Си ABI (библиотека smart_socket_ffi). */
#ifndef SMART_SOCKET_H
#define SMART_SOCKET_H

#include <stdbool.h>

typedef struct SmartSocket SmartSocket;

typedef enum SocketStatus {
    SOCKET_STATUS_OK = 0,
    SOCKET_STATUS_NULL_POINTER = 1,
    SOCKET_STATUS_CONNECTION_ERROR = 2,
    SOCKET_STATUS_TIMEOUT = 3,
    SOCKET_STATUS_PROTOCOL_ERROR = 4,
} SocketStatus;

SmartSocket *smart_socket_new_mock(double power);
SmartSocket *smart_socket_new_tcp(const char *address);
void smart_socket_free(SmartSocket *socket);

SocketStatus smart_socket_turn_on(const SmartSocket *socket);
SocketStatus smart_socket_turn_off(const SmartSocket *socket);
SocketStatus smart_socket_is_on(const SmartSocket *socket, bool *is_on);
SocketStatus smart_socket_power(const SmartSocket *socket, double *power);

#endif /* SMART_SOCKET_H */
