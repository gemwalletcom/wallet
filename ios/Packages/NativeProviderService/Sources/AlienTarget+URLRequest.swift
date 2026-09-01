// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import enum Gemstone.AlienError
import func Gemstone.alienMethodToString
import struct Gemstone.AlienTarget

extension AlienTarget {
    func asRequest() throws -> URLRequest {
        guard let url = URL(string: url) else {
            let error = AlienError.RequestError(msg: "invalid url: \(url)")
            throw error
        }
        var request = URLRequest(url: url)
        request.httpMethod = alienMethodToString(method: method)
        if let headers {
            request.allHTTPHeaderFields = headers.filter { $0.key != nativeProviderCacheHeader }
        }
        if let body {
            request.httpBody = body
        }
        return request
    }
}
