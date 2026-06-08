// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Style
import SwiftUI
import Primitives

struct SupportScrollToBottomButton: View {
    let unreadCount: Int
    let action: () -> Void

    var body: some View {
        Button(action: action) {
            Image(systemName: SystemImage.chevronDown)
                .font(Font.app.body)
                .foregroundStyle(Colors.gray)
                .frame(size: .space32 + .space6)
                .liquidGlass(fallback: { $0.background(Colors.grayVeryLight).clipShape(Circle()) })
                .overlay(alignment: .top) {
                    if unreadCount > 0 {
                        Text("\(unreadCount)")
                            .font(Font.app.footnote)
                            .foregroundStyle(Colors.whiteSolid)
                            .padding(.horizontal, .space4)
                            .frame(size: .space24)
                            .liquidGlass(tint: Colors.blue, interactive: false, in: Capsule(), fallback: { $0.background(Colors.blue).clipShape(Capsule()) })
                            .offset(y: -.space12)
                    }
                }
        }
    }
}
