// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemTransactionHeader
import Primitives
import Style
import SwiftUI

public struct TransactionHeaderView: View {
    public let header: GemTransactionHeader
    private let action: TransactionHeaderActionHandler?

    public init(
        header: GemTransactionHeader,
        action: TransactionHeaderActionHandler? = nil,
    ) {
        self.header = header
        self.action = action
    }

    public var body: some View {
        VStack(alignment: .center) {
            switch header {
            case let .amount(header):
                ValueHeaderView(
                    header: header.valueHeader,
                    isPrivacyEnabled: .constant(false),
                    titleActionType: .none,
                    spacing: .transactionAmount,
                    onHeaderAction: nil,
                    onInfoAction: nil,
                )
            case let .value(header):
                ValueHeaderView(
                    header: header.valueHeader,
                    isPrivacyEnabled: .constant(false),
                    titleActionType: .none,
                    onHeaderAction: nil,
                    onInfoAction: nil,
                )
            case let .swap(from, to):
                SwapAmountView(from: from.swapAmountField, to: to.swapAmountField, action: action)
            case let .nft(name, imageUrl):
                NftPreviewView(
                    assetImage: AssetImage(
                        type: .text("NFT"),
                        imageURL: URL(string: imageUrl),
                        placeholder: .none,
                        chainPlaceholder: .none,
                    ),
                    name: name,
                    size: .image.large,
                )
            case let .assetImage(icon):
                AssetImageView(assetImage: AssetImage(icon: icon), size: .image.large)
                    .padding(.bottom, .space12)
            }
        }
        .frame(maxWidth: .infinity)
    }
}
