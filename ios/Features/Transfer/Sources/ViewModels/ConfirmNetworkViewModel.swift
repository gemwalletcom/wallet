// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemTransferData
import enum Gemstone.TransactionInputType
import Components
import Localization
import Primitives
import PrimitivesComponents

struct ConfirmNetworkViewModel: ItemModelProvidable {
    private let transfer: GemTransferData

    init(transfer: GemTransferData) {
        self.transfer = transfer
    }
}

// MARK: - ItemModelProvidable

extension ConfirmNetworkViewModel {
    var itemModel: ConfirmTransferItemModel {
        .network(
            ListItemModel(
                title: Localized.Transfer.network,
                subtitle: networkText,
                imageStyle: .list(assetImage: AssetIdViewModel(assetId: transfer.chain.asset.id).networkAssetImage),
            ),
        )
    }
}

// MARK: - Private

extension ConfirmNetworkViewModel {
    private var networkText: String {
        let model = AssetViewModel(asset: transfer.asset)
        switch transfer.inputType {
        case .transfer, .deposit, .withdrawal:
            return model.networkFullName
        case .transferNft, .swap, .tokenApprove, .stake, .account, .generic, .perpetual, .earn:
            return model.networkName
        }
    }
}
