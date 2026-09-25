// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemConfirmHeader
import Primitives
import PrimitivesComponents

struct ConfirmHeaderViewModel {
    private let header: GemConfirmHeader
    private let currency: Currency

    init(header: GemConfirmHeader, currency: Currency) {
        self.header = header
        self.currency = currency
    }
}

// MARK: - ItemModelProvidable

extension ConfirmHeaderViewModel: ItemModelProvidable {
    var itemModel: ConfirmTransferItemModel {
        .header(headerType, isReserved: isReserved)
    }
}

// MARK: - Private

private extension ConfirmHeaderViewModel {
    var headerType: TransactionHeaderType {
        switch header {
        case let .value(value):
            .assetValue(AssetValueHeaderViewModel(data: value))
        case let .placeholder(assetId):
            .assetValue(AssetValueHeaderPlaceholder(assetImage: AssetIdViewModel(assetId: AssetId(core: assetId)).assetImage))
        case let .transaction(header), let .reserved(header):
            header.headerType(currency: currency)
        }
    }

    var isReserved: Bool {
        switch header {
        case .reserved: true
        case .placeholder, .value, .transaction: false
        }
    }
}
