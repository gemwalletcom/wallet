// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct PaymentScene: View {
    @State private var model: PaymentSceneViewModel

    public init(model: PaymentSceneViewModel) {
        _model = State(initialValue: model)
    }

    public var body: some View {
        List {
            ListAssetHeaderView(model: model.preview, subtitleLayout: .vertical)

            Section {
                ListItemImageView(
                    title: model.recipientTitle,
                    subtitle: model.quotes.merchant.name,
                    assetImage: AssetImage(imageURL: model.quotes.merchant.iconUrl?.asURL),
                )
                ListItemImageView(
                    title: model.walletTitle,
                    subtitle: model.walletText,
                    assetImage: model.walletAssetImage,
                )
            }

            if let selected = model.selectedItem {
                Section {
                    if model.showsQuoteSelection {
                        NavigationCustomLink(
                            with: payWithItem(selected),
                            action: model.onSelectQuotes,
                        )
                    } else {
                        payWithItem(selected)
                    }
                }
            }

            if let error = model.state.error {
                ListItemErrorView(errorTitle: model.errorTitle, error: error)
            }
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .safeAreaButton {
            StateButton(model.buttonModel)
        }
    }
}

// MARK: - UI Components

extension PaymentScene {
    private func payWithItem(_ item: PaymentQuoteItem) -> ListItemImageView {
        ListItemImageView(
            title: model.payWithTitle,
            subtitle: item.amountText,
            assetImage: item.assetImage,
        )
    }
}
