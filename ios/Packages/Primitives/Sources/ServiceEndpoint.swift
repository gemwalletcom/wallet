// Copyright (c). Gem Wallet. All rights reserved.

public enum ServiceEndpointType: Sendable {
    case api
    case gemNode
}

public struct ServiceEndpoint: Identifiable, Sendable {
    public let type: ServiceEndpointType
    public let host: String
    public let url: String
    public let flag: String

    public init(
        type: ServiceEndpointType,
        host: String,
        url: String,
        flag: String,
    ) {
        self.type = type
        self.host = host
        self.url = url
        self.flag = flag
    }

    public var id: String {
        url
    }
}
