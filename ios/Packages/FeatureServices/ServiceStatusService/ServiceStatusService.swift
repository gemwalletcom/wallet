// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import class Gemstone.GemServiceStatus
import func Gemstone.serviceStatusTimeoutSeconds
import NativeProviderService
import Primitives

internal import GemstonePrimitives

public struct ServiceStatusService: ServiceStatusServiceable, Sendable {
    private let client: GemServiceStatus

    public init(requestInterceptor: any RequestInterceptable) {
        let configuration = URLSessionConfiguration.default
        configuration.timeoutIntervalForRequest = TimeInterval(serviceStatusTimeoutSeconds())

        client = GemServiceStatus(
            provider: NativeProvider(
                session: URLSession(configuration: configuration),
                url: Constants.apiURL,
                requestInterceptor: requestInterceptor,
            ),
        )
    }

    public var endpoints: [ServiceEndpoint] {
        client.getEndpoints().map { $0.map() }
    }

    public func endpointLatency(url: String) async throws -> UInt64 {
        try await client.getEndpointLatency(url: url)
    }
}
