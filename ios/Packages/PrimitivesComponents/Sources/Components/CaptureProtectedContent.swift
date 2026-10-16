// Copyright (c). Gem Wallet. All rights reserved.

import Localization
import Style
import SwiftUI

struct CaptureProtectedContent<Content: View>: View {
    var content: Content
    let isProtected: Bool

    var body: some View {
        if isProtected {
            content
        } else {
            Text(Localized.Common.notAvailable)
                .font(.callout)
                .foregroundStyle(Colors.secondaryText)
                .frame(maxWidth: .infinity)
        }
    }
}
