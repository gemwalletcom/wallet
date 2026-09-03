// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.AlienError
import protocol Gemstone.AlienProvider
import class Gemstone.AlienResponse
import struct Gemstone.AlienTarget
import typealias Gemstone.Chain
import Primitives

public actor NativeProvider {
    private let session: URLSession
    private let nodeProvider: any NodeURLProvidable
    private let cache: MemoryCache
    private let requestInterceptor: any RequestInterceptable

    public init(
        session: URLSession = .shared,
        nodeProvider: any NodeURLProvidable,
        requestInterceptor: any RequestInterceptable = EmptyRequestInterceptor(),
    ) {
        self.session = session
        self.nodeProvider = nodeProvider
        self.cache = MemoryCache()
        self.requestInterceptor = requestInterceptor
    }

    public init(session: URLSession = .shared, requestInterceptor: any RequestInterceptable = EmptyRequestInterceptor()) {
        self.init(
            session: session,
            nodeProvider: ApiOnlyNodes(),
            requestInterceptor: requestInterceptor,
        )
    }
}

struct ApiOnlyNodes: NodeURLProvidable {
    func node(for chain: Primitives.Chain) -> URL {
        preconditionFailure("API-only provider asked for a \(chain) node")
    }
}

extension NativeProvider: AlienProvider {
    public func request(target: AlienTarget) async throws -> AlienResponse {
        let cacheKey = target.nativeCacheKey
        if let cacheKey, let data = await cache.get(key: cacheKey) {
            return AlienResponse(status: 200, data: data)
        }
        do {
            var request = try target.asRequest()
            requestInterceptor.intercept(request: &request)
            let (data, response) = try await session.data(for: request)
            let statusCode = (response as? HTTPURLResponse)?.statusCode

            if let cacheKey {
                await cache.set(key: cacheKey, value: data)
            }

            return AlienResponse(status: statusCode.map(UInt16.init), data: data)
        } catch {
            if isNetworkError(error) {
                throw AlienError.Offline
            }
            if (error as NSError).domain == NSURLErrorDomain {
                throw AlienError.ResponseError(msg: error.localizedDescription)
            }
            throw error
        }
    }

    public nonisolated func getEndpoint(chain: Chain) throws -> String {
        try nodeProvider.node(for: Primitives.Chain(id: chain)).absoluteString
    }
}
