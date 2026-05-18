// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import func Gemstone.alienMethodToString
import struct Gemstone.AlienTarget

internal import CryptoKit

let nativeProviderCacheHeader = "x-gem-cache-ttl"
private let cacheKeySeparator = Data([0])

extension AlienTarget {
    var nativeCacheKey: String? {
        guard headers?[nativeProviderCacheHeader] != nil else {
            return nil
        }
        var hasher = SHA256()
        hasher.update(data: Data(alienMethodToString(method: method).utf8))
        hasher.update(data: cacheKeySeparator)
        hasher.update(data: Data(url.utf8))
        if let body {
            hasher.update(data: cacheKeySeparator)
            hasher.update(data: body)
        }
        return Data(hasher.finalize()).base64EncodedString()
    }
}
