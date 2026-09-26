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
            if model.showFeeAssets, let selectedFeeAsset = model.selectedFeeAssetItem {
                Section {
                    NavigationCustomLink(
                        with: ListItemView(model: selectedFeeAsset.listItem),
                        action: { isPresentingFeeAssetSelection = true },
                    )
                } header: {
                    Text(Localized.Swap.youPay)
                        .listRowInsets(.horizontalMediumInsets)
                }
            }

            if model.showFeeRates {
                Section {
                    ForEach(model.feeRateRows, id: \.title) { row in
                        NavigationCustomLink(
                            with: FeeRow(
                                emoji: row.emoji,
                                isSelected: row.isSelected,
                                model: model.rowItem(for: row),
                            ),
                        ) {
                            switch row.kind {
                            case let .priority(priority):
                                model.select(.priority(priority: priority))
                                dismiss()
                            case .custom:
                                isPresentingCustomFee = true
                            }
                        }
                    }
                } footer: {
                    Text(model.infoIcon)
                        .textStyle(.caption)
                        .multilineTextAlignment(.leading)
                        .headerProminence(.increased)
                }
            }

            if model.feeItems.isEmpty {
                ListItemView(model: model.feeListItem)
            } else {
                Section {
                    ForEach(model.feeItems, id: \.title) {
                        ListItemView(model: $0)
                    }
                    ListItemView(model: model.feeListItem)
                }
            }
        }
        .contentMargins(.top, .scene.top, for: .scrollContent)
        .navigationTitle(model.title)
        .navigationBarTitleDisplayMode(.inline)
        .toolbarDismissItem(type: .confirm, placement: .topBarTrailing)
        .navigationDestination(isPresented: $isPresentingCustomFee) {
            if let customFeeModel = model.customFeeModel() {
                NetworkFeeCustomScene(
                    model: customFeeModel,
                    onConfirm: { dismiss() },
                )
            }
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
                listContent: { ListItemView(model: $0.listItem) },
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
