// Copyright (c). Gem Wallet. All rights reserved.

import Foundation

public enum ImageSource: Sendable, Equatable, Hashable {
    case remote(URL)
    case local(fileName: String)

    public init(_ value: String) {
        if let url = value.asURL, url.scheme != nil {
            self = .remote(url)
        } else {
            self = .local(fileName: value)
        }
    }

    public var url: URL {
        switch self {
        case let .remote(url): url
        case let .local(fileName): URL.documentsDirectory.appendingPathComponent(fileName)
        }
    }
}
