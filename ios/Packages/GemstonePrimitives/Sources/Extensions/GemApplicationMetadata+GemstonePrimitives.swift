// Copyright (c). Gem Wallet. All rights reserved.

import Gemstone
import Primitives

public extension GemApplicationMetadata {
    func map() -> ApplicationMetadata {
        ApplicationMetadata(
            name: name,
            description: description,
            url: url,
            icon: icon,
            source: source.map(),
        )
    }
}

public extension ApplicationMetadata {
    func map() -> GemApplicationMetadata {
        GemApplicationMetadata(
            name: name,
            description: description,
            url: url,
            icon: icon,
            source: source.map(),
        )
    }

    var shortName: String {
        Gemstone.applicationMetadataShortName(metadata: map())
    }
}

private extension GemApplicationMetadataSource {
    func map() -> ApplicationMetadataSource {
        switch self {
        case .walletConnect: .walletConnect
        case .payment: .payment
        }
    }
}

private extension ApplicationMetadataSource {
    func map() -> GemApplicationMetadataSource {
        switch self {
        case .walletConnect: .walletConnect
        case .payment: .payment
        }
    }
}
