// Copyright (c). Gem Wallet. All rights reserved.

import Foundation
import Primitives
import Transfer

public struct TransferMetadataProviderMock: TransferMetadataProvidable {
    public var metadataResult: Result<TransferDataMetadata, Error>

    public init(metadataResult: Result<TransferDataMetadata, Error>) {
        self.metadataResult = metadataResult
    }

    public func metadata(
        walletId _: WalletId,
        asset _: Asset,
        extraIds _: [AssetId],
    ) throws -> TransferDataMetadata {
        try metadataResult.get()
    }
}
