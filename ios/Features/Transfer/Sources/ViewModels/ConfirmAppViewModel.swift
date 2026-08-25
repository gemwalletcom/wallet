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
        type.applicationMetadata?.shortName
    }

    private var assetImage: AssetImage? {
        type.applicationMetadata.map { AssetImage(imageURL: $0.icon.asURL) }
    }
}
