// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

public struct ConfirmAppViewModel: ItemModelProvidable {
    private let type: TransferDataType

    init(type: TransferDataType) {
        self.type = type
    }
}

// MARK: - ItemModelPrividable

public extension ConfirmAppViewModel {
    var itemModel: ConfirmTransferItemModel {
        guard let name = appValue else { return .empty }

        return .app(
            ListItemModel(
                title: title,
                subtitle: name,
                imageStyle: .list(assetImage: assetImage),
            ),
        )
    }
}

// MARK: - Private

extension ConfirmAppViewModel {
    private var title: String {
        switch type {
        case .payment: Localized.Transfer.merchant
        default: Localized.WalletConnect.app
        }
    }

    private var appValue: String? {
        switch type {
        case .transfer,
             .deposit,
             .withdrawal,
             .transferNft,
             .swap,
             .tokenApprove,
             .stake,
             .account,
             .perpetual,
             .earn: .none
        case let .generic(_, app, _):
            app.shortName
        case let .payment(_, payment, _):
            payment.merchant.name
        }
    }

    private var assetImage: AssetImage? {
        switch type {
        case .transfer,
             .deposit,
             .withdrawal,
             .transferNft,
             .swap,
             .tokenApprove,
             .stake,
             .account,
             .perpetual,
             .earn:
            .none
        case let .generic(_, app, _):
            AssetImage(imageURL: app.icon?.asURL)
        case let .payment(_, payment, _):
            AssetImage(imageURL: payment.merchant.iconUrl?.asURL)
        }
    }
}
