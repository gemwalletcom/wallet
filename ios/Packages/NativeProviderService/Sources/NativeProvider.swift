// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.AlienError
import protocol Gemstone.AlienProvider
import class Gemstone.AlienResponse
import struct Gemstone.AlienTarget
import Primitives

public actor NativeProvider {
    private let session: URLSession

    public init(session: URLSession = .shared) {
        self.session = session
    }
}

extension NativeProvider: AlienProvider {
    public func request(target: AlienTarget) async throws -> AlienResponse {
        do {
            let request = try target.asRequest()
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
