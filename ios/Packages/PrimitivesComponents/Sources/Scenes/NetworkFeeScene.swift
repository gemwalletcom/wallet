// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
import SwiftUI

public struct NetworkFeeScene: View {
    @Environment(\.dismiss) private var dismiss

    private var model: NetworkFeeSceneViewModel

    @State private var isPresentingCustomFee = false

    public init(model: NetworkFeeSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        List {
            if model.showFeeRates {
                Section {
                    ForEach(model.feeRatesViewModels) { feeRate in
                        Button {
                            model.onSelectPreset(feeRate)
                            dismiss()
                        } label: {
                            HStack(spacing: .space12) {
                                EmojiView(
                                    color: Colors.grayBackground,
                                    emoji: feeRate.emoji,
                                )
                                .frame(width: Sizing.image.asset, height: Sizing.image.asset)
                                .assetBadge(model.isSelected(feeRate) ? Images.Wallets.selected : nil)

                                ListItemView(
                                    title: feeRate.title,
                                    subtitle: model.valueForRate(feeRate),
                                    subtitleStyle: .init(font: .callout, color: Colors.black, fontWeight: .medium),
                                    subtitleExtra: model.fiatValueForRate(feeRate),
                                    subtitleStyleExtra: .init(font: .footnote, color: Colors.gray),
                                )
                            }
                        }
                    }

                    if model.isCustomSelected {
                        Button {
                            isPresentingCustomFee = true
                        } label: {
                            HStack(spacing: .space12) {
                                EmojiView(color: Colors.grayBackground, emoji: model.customFeeEmoji)
                                    .frame(width: Sizing.image.asset, height: Sizing.image.asset)
                                    .assetBadge(Images.Wallets.selected)

                                ListItemView(
                                    title: model.customFeeTitle,
                                    subtitle: model.customRateText,
                                    subtitleStyle: .init(font: .callout, color: Colors.black, fontWeight: .medium),
                                    subtitleExtra: model.fiatValue,
                                    subtitleStyleExtra: .init(font: .footnote, color: Colors.gray),
                                )
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
        .toolbar {
            ToolbarItem(placement: .cancellationAction) {
                Button("", systemImage: SystemImage.xmark, action: { dismiss() })
            }
            if model.supportsCustomFee {
                ToolbarItem(placement: .topBarTrailing) {
                    Button("", systemImage: SystemImage.settings) { isPresentingCustomFee = true }
                }
            }
        }
        .navigationDestination(isPresented: $isPresentingCustomFee) {
            NetworkFeeCustomScene(model: model.customFeeModel(onComplete: { dismiss() }))
        }
    }
}
