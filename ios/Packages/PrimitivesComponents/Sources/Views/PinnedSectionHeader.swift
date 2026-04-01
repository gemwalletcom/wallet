// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Style
import SwiftUI

public struct PinnedSectionHeader: View {
    public init() {}

    public var body: some View {
        SectionHeaderView(
            title: Localized.Common.pinned,
            image: Images.System.pin,
        )
    }
}
