// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.AlienError
import protocol Gemstone.AlienProvider
import class Gemstone.AlienResponse
import struct Gemstone.AlienTarget
import Primitives

public actor NativeProvider {
    private let session: URLSession
    private let requestInterceptor: any RequestInterceptable

    public init(session: URLSession = .shared, requestInterceptor: any RequestInterceptable = EmptyRequestInterceptor()) {
        self.session = session
        self.requestInterceptor = requestInterceptor
    }
}

extension NativeProvider: AlienProvider {
    public func request(target: AlienTarget) async throws -> AlienResponse {
        do {
            var request = try target.asRequest()
            requestInterceptor.intercept(request: &request)
            let (data, response) = try await session.data(for: request)
            let statusCode = (response as? HTTPURLResponse)?.statusCode

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
}
