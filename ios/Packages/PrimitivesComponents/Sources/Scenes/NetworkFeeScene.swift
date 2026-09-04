// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import Style
import SwiftUI

public struct NetworkFeeScene: View {
    @Environment(\.dismiss) private var dismiss

    private var model: NetworkFeeSceneViewModel

    @State private var isPresentingCustomFee = false
    @State private var isPresentingFeeAssetSelection = false

    public init(model: NetworkFeeSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        List {
            if model.showFeeAssets {
                Section {
                    NavigationCustomLink(
                        with: SimpleListItemView(model: model.selectedFeeAssetItem),
                        action: { isPresentingFeeAssetSelection = true },
                    )
                } header: {
                    Text(Localized.Swap.youPay)
                        .listRowInsets(.horizontalMediumInsets)
                }
            }

            if model.showFeeRates {
                Section {
                    ForEach(model.feeRatesViewModels) { feeRate in
                        NavigationCustomLink(
                            with: FeeRow(
                                emoji: feeRate.emoji,
                                isSelected: model.isSelected(feeRate),
                                model: model.rowItem(for: feeRate),
                            ),
                        ) {
                            model.select(.priority(priority: feeRate.priority.map()))
                            dismiss()
                        }
                    }

                    if model.supportsCustomFee {
                        NavigationCustomLink(
                            with: FeeRow(
                                emoji: Emoji.FeeRate.custom.rawValue,
                                isSelected: model.isCustomSelected,
                                model: model.customRowItem,
                            ),
                        ) {
                            isPresentingCustomFee = true
                        }
                    }
                } footer: {
                    Text(model.infoIcon)
                        .textStyle(.caption)
                        .multilineTextAlignment(.leading)
                        .headerProminence(.increased)
                }
            }

            ListItemView(
                title: model.title,
                subtitle: model.value,
                subtitleExtra: model.fiatValue,
                placeholders: [.subtitle],
            )
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .toolbarDismissItem(type: .confirm, placement: .topBarTrailing)
        .navigationDestination(isPresented: $isPresentingCustomFee) {
            NetworkFeeCustomScene(
                model: model.customFeeModel(),
                onConfirm: { dismiss() },
            )
        }
        .sheet(isPresented: $isPresentingFeeAssetSelection) {
            SelectableListNavigationStack(
                model: model.feeAssetsViewModel,
                onFinishSelection: {
                    if let item = $0.first {
                        model.selectFeeAsset(item)
                    }
                    isPresentingFeeAssetSelection = false
                },
                listContent: { SimpleListItemView(model: $0) },
            )
        }
    }
}

private struct FeeRow: View {
    let emoji: String
    let isSelected: Bool
    let model: ListItemModel

    var body: some View {
        HStack(spacing: .space12) {
            EmojiView(color: Colors.grayBackground, emoji: emoji)
                .frame(width: Sizing.image.asset, height: Sizing.image.asset)
                .assetBadge(isSelected ? Images.Wallets.selected : nil)

            ListItemView(model: model)
        }
    }
}
