// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Gemstone
import Primitives

public extension GemNftType {
    func map() -> NFTType {
        switch self {
        case .erc721: .erc721
        case .erc1155: .erc1155
        case .spl: .spl
        case .jetton: .jetton
        }
    }
}

public extension NFTType {
    func map() -> GemNftType {
        switch self {
        case .erc721: .erc721
        case .erc1155: .erc1155
        case .spl: .spl
        case .jetton: .jetton
        }
    }
}

public extension GemNftAttributeType {
    func map() -> NFTAttributeType {
        switch self {
        case .string: .string
        case .timestamp: .timestamp
        }
    }
}

public extension NFTAttributeType {
    func map() -> GemNftAttributeType {
        switch self {
        case .string: .string
        case .timestamp: .timestamp
        }
    }
}

public extension GemNftResource {
    func map() -> NFTResource {
        NFTResource(url: url, mimeType: mimeType)
    }
}

public extension NFTResource {
    func map() -> GemNftResource {
        GemNftResource(url: url, mimeType: mimeType)
    }
}

public extension GemNftImages {
    func map() -> NFTImages {
        NFTImages(preview: preview.map())
    }
}

public extension NFTImages {
    func map() -> GemNftImages {
        GemNftImages(preview: preview.map())
    }
}

public extension GemNftAttribute {
    func map() -> NFTAttribute {
        NFTAttribute(name: name, value: value, valueType: valueType?.map(), percentage: percentage)
    }
}

public extension NFTAttribute {
    func map() -> GemNftAttribute {
        GemNftAttribute(name: name, value: value, valueType: valueType?.map(), percentage: percentage)
    }
}

public extension GemNftAsset {
    func map() throws -> NFTAsset {
        try NFTAsset(
            id: id,
            collectionId: collectionId,
            contractAddress: contractAddress,
            tokenId: tokenId,
            tokenType: tokenType.map(),
            name: name,
            description: description,
            chain: chain.map(),
            resource: resource.map(),
            images: images.map(),
            attributes: attributes.map { $0.map() },
        )
    }
}

public extension NFTAsset {
    func map() -> GemNftAsset {
        GemNftAsset(
            id: id,
            collectionId: collectionId,
            contractAddress: contractAddress,
            tokenId: tokenId,
            tokenType: tokenType.map(),
            name: name,
            description: description,
            chain: chain.rawValue,
            resource: resource.map(),
            images: images.map(),
            attributes: attributes.map { $0.map() },
        )
    }
}
