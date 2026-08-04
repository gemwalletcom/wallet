// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import PrimitivesComponents
import Style
import SwiftUI

public struct PaymentQuotesScene: View {
    @State private var model: PaymentQuotesSceneViewModel
    private let onComplete: () -> Void

    public init(
        model: PaymentQuotesSceneViewModel,
        onComplete: @escaping () -> Void,
    ) {
        _model = State(wrappedValue: model)
        self.onComplete = onComplete
    }

    public var body: some View {
        List {
            ListAssetHeaderView(model: model.preview, subtitleLayout: .vertical)

            Section {
                ListItemImageView(
                    title: model.merchantTitle,
                    subtitle: model.merchantText,
                    assetImage: model.merchantAssetImage,
                )
                ListItemImageView(
                    title: Localized.Common.wallet,
                    subtitle: model.walletText,
                    assetImage: model.walletAssetImage,
                )
                if let expiresAt = model.expiresAt {
                    ListItemExpiryView(
                        title: model.expiresTitle,
                        expiresAt: expiresAt,
                    )
                }
            }

            if let selected = model.selectedItem {
                Section {
                    NavigationCustomLink(
                        with: ListItemImageView(
                            title: model.quotesTitle,
                            subtitle: selected.amountText,
                            assetImage: selected.assetImage,
                        ),
                        action: model.onSelectQuotes,
                    )
                }
            }
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .listSectionSpacing(.compact)
        .taskOnce { model.onAppear() }
        .task { await model.awaitExpiry() }
        .safeAreaButton {
            StateButton(
                text: model.buttonTitle,
                type: model.buttonType,
                action: confirm,
            )
        }
        .sheet(isPresented: $model.isPresentingQuotes) {
            SelectableListNavigationStack(
                model: model.quotesModel,
                onFinishSelection: model.onFinishQuotesSelection,
                listContent: { ListAssetItemView(model: $0) },
            )
        }
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
    }

    private func confirm() {
        model.onConfirm()
        onComplete()
    }
}
