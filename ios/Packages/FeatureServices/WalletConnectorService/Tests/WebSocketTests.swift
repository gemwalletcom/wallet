// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Testing
@testable import WalletConnectorService
@testable import WalletConnectorServiceTestKit

struct WebSocketTests {
    @Test
    func initialState() {
        let socket = WebSocket.mock()

        #expect(socket.isConnected == false)
        #expect(socket.onConnect == nil)
        #expect(socket.onDisconnect == nil)
        #expect(socket.onText == nil)
    }

    @Test
    func writeWithoutConnectionCallsCompletionImmediately() async {
        let socket = WebSocket.mock()

        await confirmation { confirm in
            socket.write(string: "test") {
                confirm()
            }
        }
    }

    @Test
    func aFailedHandshakeReportsOneDisconnectAndTheNextAttemptCanReportAgain() throws {
        let socket = try WebSocket(request: URLRequest(url: #require(URL(string: "wss://127.0.0.1:9"))))
        let disconnects = Locked(wrappedValue: 0)
        socket.onDisconnect = { _ in disconnects.withLock { $0 += 1 } }

        socket.connect()
        let first = try #require(socket.task)
        socket.urlSession(URLSession.shared, task: first, didCompleteWithError: URLError(.cannotConnectToHost))
        socket.urlSession(URLSession.shared, webSocketTask: first, didCloseWith: .abnormalClosure, reason: nil)
        socket.urlSession(URLSession.shared, task: first, didCompleteWithError: URLError(.cannotConnectToHost))
        #expect(disconnects.wrappedValue == 1)
        #expect(socket.isConnected == false)

        socket.connect()
        let retry = try #require(socket.task)
        socket.urlSession(URLSession.shared, task: first, didCompleteWithError: URLError(.cannotConnectToHost))
        #expect(disconnects.wrappedValue == 1, "a late callback from the replaced attempt is ignored")

        socket.urlSession(URLSession.shared, task: retry, didCompleteWithError: URLError(.cannotConnectToHost))
        #expect(disconnects.wrappedValue == 2)
        socket.disconnect()
    }

    // MARK: - Stress Tests

    @Test
    func stressTestConcurrentPropertyAccess() async {
        let socket = WebSocket.mock()
        let iterations = 1000

        await withTaskGroup(of: Void.self) { group in
            group.addTask {
                for i in 0 ..< iterations {
                    socket.request = URLRequest(url: URL(string: "wss://test\(i).com")!)
                }
            }

            group.addTask {
                for _ in 0 ..< iterations {
                    _ = socket.request
                    _ = socket.isConnected
                }
            }

            group.addTask {
                for _ in 0 ..< iterations {
                    socket.onConnect = {}
                    socket.onDisconnect = { _ in }
                    socket.onText = { _ in }
                }
            }
        }
    }

    @Test
    func stressTestConcurrentWrites() async {
        let socket = WebSocket.mock()
        let iterations = 100

        await confirmation(expectedCount: iterations) { confirm in
            await withTaskGroup(of: Void.self) { group in
                for i in 0 ..< iterations {
                    group.addTask {
                        socket.write(string: "message\(i)") {
                            confirm()
                        }
                    }
                }
            }
        }
    }

    @Test
    func stressTestConcurrentConnectDisconnect() async {
        let socket = WebSocket.mock()
        let iterations = 50

        await withTaskGroup(of: Void.self) { group in
            group.addTask {
                for _ in 0 ..< iterations {
                    socket.connect()
                }
            }

            group.addTask {
                for _ in 0 ..< iterations {
                    socket.disconnect()
                }
            }

            group.addTask {
                for _ in 0 ..< iterations {
                    _ = socket.isConnected
                }
            }
        }
    }
}
