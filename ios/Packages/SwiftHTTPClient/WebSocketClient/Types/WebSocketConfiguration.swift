// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public struct WebSocketConfiguration: Sendable {
    public typealias SessionFactory = @Sendable (URLSessionConfiguration, any URLSessionDelegate) -> URLSession

    public let requestProvider: any WebSocketRequestProvider
    public let reconnection: any Reconnectable
    public let sessionConfiguration: URLSessionConfiguration
    public let makeSession: SessionFactory

    public init(
        requestProvider: any WebSocketRequestProvider,
        reconnection: any Reconnectable,
        sessionConfiguration: URLSessionConfiguration = .default,
        makeSession: @escaping SessionFactory = { URLSession(configuration: $0, delegate: $1, delegateQueue: nil) },
    ) {
        self.requestProvider = requestProvider
        self.reconnection = reconnection
        self.sessionConfiguration = sessionConfiguration
        self.makeSession = makeSession
    }

    public init(
        url: URL,
        reconnection: any Reconnectable,
        sessionConfiguration: URLSessionConfiguration = .default,
    ) {
        self.init(requestProvider: StaticRequestProvider(url: url), reconnection: reconnection, sessionConfiguration: sessionConfiguration)
    }
}
