// Copyright (c). Gem Wallet. All rights reserved.

import enum Gemstone.GemAssetSectionKind
import SwiftUI

public struct PinnedSectionHeader: View {
    public init() {}

    public var body: some View {
        SectionHeaderView(
            title: GemAssetSectionKind.pinned.title ?? "",
            image: GemAssetSectionKind.pinned.image,
        )
    }
}
