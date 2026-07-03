// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
import SwiftUI

public enum TransactionHeaderType {
    case amount(AmountDisplay)
    case swap(from: SwapAmountField, to: SwapAmountField)
    case nft(name: String?, image: AssetImage)
    case asset(image: AssetImage)
    case assetValue(AssetValueHeaderData)
}

public struct TransactionHeaderView: View {
    public let type: TransactionHeaderType
    private let action: TransactionHeaderActionHandler?

    public init(
        type: TransactionHeaderType,
        action: TransactionHeaderActionHandler? = nil,
    ) {
        self.type = type
        self.action = action
    }

    public var body: some View {
        VStack(alignment: .center) {
            switch type {
            case let .amount(display):
                ValueHeaderView(
                    model: TransactionAmountHeaderViewModel(display: display),
                    isPrivacyEnabled: .constant(false),
                    titleActionType: .none,
                    spacing: .transactionAmount,
                    onHeaderAction: nil,
                    onInfoAction: nil,
                )
            case let .swap(from, to):
                SwapAmountView(from: from, to: to, action: action)
            case let .nft(name, image):
                NftPreviewView(assetImage: image, name: name, size: .image.large)
            case let .asset(image):
                AssetImageView(assetImage: image, size: .image.large)
                    .padding(.bottom, .space12)
            case let .assetValue(data):
                ValueHeaderView(
                    model: AssetValueHeaderViewModel(data: data),
                    isPrivacyEnabled: .constant(false),
                    titleActionType: .none,
                    onHeaderAction: nil,
                    onInfoAction: nil,
                )
            }
        }
        .frame(maxWidth: .infinity)
    }
}
