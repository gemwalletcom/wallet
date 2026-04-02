// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Localization
import Primitives
import Style
import SwiftUI

public struct NetworkFeeScene: View {
    @Environment(\.dismiss) private var dismiss

    private var model: NetworkFeeSceneViewModel

    public init(model: NetworkFeeSceneViewModel) {
        self.model = model
    }

    public var body: some View {
        List {
            if model.showFeeRates {
                Section {
                    ForEach(model.feeRatesViewModels) { feeRate in
                        let isSelected = feeRate.feeRate.priority == model.priority
                        Button {
                            model.priority = feeRate.feeRate.priority
                            dismiss()
                        } label: {
                            HStack(spacing: .space12) {
                                EmojiView(
                                    color: Colors.grayBackground,
                                    emoji: feeRate.emoji,
                                )
                                .frame(width: Sizing.image.asset, height: Sizing.image.asset)
                                .assetBadge(isSelected ? Images.Wallets.selected : nil)

                                ListItemView(
                                    title: feeRate.title,
                                    subtitle: feeRate.valueText,
                                    subtitleStyle: .init(font: .callout, color: Colors.black, fontWeight: .medium),
                                    subtitleExtra: model.fiatValueForRate(feeRate),
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
        }
    }
}
