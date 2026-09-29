// Copyright (c). Gem Wallet. All rights reserved.

import Primitives

public extension AssetId {
    static func mock(
        chain: Chain = .bitcoin,
        tokenId: String? = nil,
    ) -> AssetId {
        AssetId(chain: chain, tokenId: tokenId)
    }
}

public extension WalletId {
    static func mock(
        address: String = "0x0000000000000000000000000000000000000000",
    ) -> WalletId {
        .multicoin(address: address)
    }
}

public extension TransactionId {
    static func mock(
        chain: Chain = .bitcoin,
        hash: String = "tx-id",
    ) -> TransactionId {
        TransactionId(chain: chain, hash: hash)
    }
}

public extension NFTAssetId {
    static func mock(
        chain: Chain = .bitcoin,
        contractAddress: String = "0xcontract",
        tokenId: String = "1",
    ) -> NFTAssetId {
        NFTAssetId(chain: chain, contractAddress: contractAddress, tokenId: tokenId)
    }
}

public extension NFTCollectionId {
    static func mock(
        chain: Chain = .bitcoin,
        contractAddress: String = "0xcontract",
    ) -> NFTCollectionId {
        NFTCollectionId(chain: chain, contractAddress: contractAddress)
    }
}

public extension PerpetualId {
    static func mock(
        provider: PerpetualProvider = .hypercore,
        symbol: String = "BTC",
    ) -> PerpetualId {
        PerpetualId(provider: provider, symbol: symbol)
    }
}

public extension AssetAddress {
    static func mock(
        asset: Asset = .mock(),
        address: String = "",
    ) -> AssetAddress {
        AssetAddress(
            asset: asset,
            address: address,
        )
    }
}
