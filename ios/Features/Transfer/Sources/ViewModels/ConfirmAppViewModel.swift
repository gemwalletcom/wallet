// Copyright (c). Gem Wallet. All rights reserved.

import struct Gemstone.GemTransferData
import Components
import Foundation
import GemstonePrimitives
import Localization
import Primitives
import PrimitivesComponents

public struct ConfirmAppViewModel: ItemModelProvidable {
    private let transfer: GemTransferData

    init(transfer: GemTransferData) {
        self.transfer = transfer
    }
}

// MARK: - ItemModelProvidable

public extension ConfirmAppViewModel {
    var itemModel: ConfirmTransferItemModel {
        guard let name = appValue else { return .empty }

        return .app(
            ListItemModel(
                title: Localized.WalletConnect.app,
                subtitle: name,
                imageStyle: .list(assetImage: assetImage),
            ),
        )
    }
}

// MARK: - Private

extension ConfirmAppViewModel {
    private var appValue: String? {
        transfer.applicationShortName()
    }

    private var assetImage: AssetImage? {
        transfer.applicationMetadata.map { AssetImage(imageURL: $0.icon.asURL) }
    }
}
