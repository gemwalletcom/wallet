// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Primitives
import PrimitivesComponents
import SwiftUI

public struct AvatarView: View {
    let avatarImage: AssetImage
    let size: CGFloat
    let action: VoidAction

    public init(
        avatarImage: AssetImage,
        size: CGFloat,
        action: VoidAction,
    ) {
        self.avatarImage = avatarImage
        self.size = size
        self.action = action
    }

    public var body: some View {
        Button {
            action?()
        } label: {
            AssetImageView(
                assetImage: avatarImage,
                size: size,
            )
        }
    }
}
