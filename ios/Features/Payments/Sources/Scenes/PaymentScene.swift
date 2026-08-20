// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
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
                    title: model.appTitle,
                    subtitle: model.quotes.merchant.name,
                    assetImage: AssetImage(imageURL: model.quotes.merchant.iconUrl?.asURL),
                )
                ListItemImageView(
                    title: model.walletTitle,
                    subtitle: model.walletText,
                    assetImage: model.walletAssetImage,
                )
                if let expiresAt = model.quotes.expiresAt {
                    ListItemExpiryView(title: model.expiresTitle, expiresAt: expiresAt)
                }
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
                } footer: {
                    if let text = model.verificationText {
                        verificationFooter(text)
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
        .task(id: model.state.quotes.expiresAt) { await model.awaitExpiry() }
    }
}

// MARK: - UI Components

extension PaymentScene {
    private func verificationFooter(_ text: String) -> some View {
        HStack(alignment: .top, spacing: .space4) {
            InfoButton(action: model.onSelectVerificationInfo)
            Text(text)
        }
    }

    private func payWithItem(_ item: PaymentQuoteItem) -> ListItemImageView {
        ListItemImageView(
            title: model.payWithTitle,
            subtitle: item.amountText,
            assetImage: item.assetImage,
        )
    }
}
