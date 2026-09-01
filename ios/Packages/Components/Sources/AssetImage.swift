// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public enum AssetImageType: Sendable, Equatable, Hashable {
    case text(String)
    case emoji(String)

    public var value: String {
        switch self {
        case let .text(value), let .emoji(value):
            value
        }
    }
}

public struct AssetImage: Sendable, Equatable {
    public let type: AssetImageType?
    public let imageURL: URL?
    public let placeholder: Image?
    public let chainPlaceholder: Image?

    public init(
        type: AssetImageType? = .none,
        imageURL: URL? = .none,
        placeholder: Image? = .none,
        chainPlaceholder: Image? = .none,
    ) {
        self.type = type
        self.imageURL = imageURL
        self.placeholder = placeholder
        self.chainPlaceholder = chainPlaceholder
    }

    public static func image(_ image: Image) -> AssetImage {
        AssetImage(
            type: .none,
            imageURL: .none,
            placeholder: image,
            chainPlaceholder: .none,
        )
    }
}
