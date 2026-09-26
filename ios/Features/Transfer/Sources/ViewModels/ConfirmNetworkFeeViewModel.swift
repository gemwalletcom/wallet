// Copyright (c). Gem Wallet. All rights reserved.

import Components
import struct Gemstone.GemConfirmFeeRow
import enum Gemstone.GemInfoTopic
import Primitives
import PrimitivesComponents

struct ConfirmNetworkFeeViewModel {
    private let feeRow: GemConfirmFeeRow
    private let onInfo: (GemInfoTopic) -> Void

    init(
        feeRow: GemConfirmFeeRow,
        onInfo: @escaping (GemInfoTopic) -> Void,
    ) {
        self.feeRow = feeRow
        self.onInfo = onInfo
    }
}

// MARK: - Item Model

extension ConfirmNetworkFeeViewModel {
    var itemModel: ConfirmTransferItemModel {
        .networkFee(
            .init(
                title: feeRow.title.text,
                subtitle: networkFeeValue,
                subtitleExtra: networkFeeExtra,
                placeholders: [.subtitle],
                infoAction: { [feeRow, onInfo] in onInfo(feeRow.info) },
            ),
            selectable: feeRow.opensDetails,
        )
    }
}

// MARK: - Private

extension ConfirmNetworkFeeViewModel {
    private var networkFeeValue: String? {
        switch feeRow.value {
        case let .unavailable(text): text
        case .loading: nil
        case let .ready(text): text.value.text()
        }
    }

    private var networkFeeExtra: String? {
        switch feeRow.value {
        case .loading, .unavailable: nil
        case let .ready(text): text.extra?.text
        }
    }
}
