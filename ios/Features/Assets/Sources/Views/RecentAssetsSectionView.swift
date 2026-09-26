// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct RecentAssetsSectionView: View {
    private let model: RecentAssetsViewModel
    private let onSelect: (Asset) -> Void

    public init(
        model: RecentAssetsViewModel,
        onSelect: @escaping (Asset) -> Void,
    ) {
        self.model = model
        self.onSelect = onSelect
    }

    public var body: some View {
        Section {} header: {
            VStack(alignment: .leading, spacing: Spacing.space12) {
                SectionHeaderView(title: Localized.RecentActivity.title, action: model.present)
                    .padding(.leading, Spacing.space12)
                AssetsCollectionView(models: model.assetTexts) { text in
                    Button {
                        onSelect(text.asset.toPrimitives())
                    } label: {
                        AssetChipView(model: text)
                    }
                }
            }
            .padding(.top, Spacing.small)
            .padding(.bottom, Spacing.tiny)
        }
        .textCase(nil)
        .listRowInsets(EdgeInsets())
    }
}
