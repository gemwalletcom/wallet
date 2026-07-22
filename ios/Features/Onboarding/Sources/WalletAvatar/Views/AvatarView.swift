// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Foundation
import Primitives
import PrimitivesComponents
import Style
import SwiftUI

public struct AvatarView: View {
    let avatarImage: AssetImage
    let size: CGFloat
    let action: VoidAction
    let removeAction: VoidAction

    public init(
        avatarImage: AssetImage,
        size: CGFloat,
        action: VoidAction = nil,
        removeAction: VoidAction = nil,
    ) {
        self.avatarImage = avatarImage
        self.size = size
        self.action = action
        self.removeAction = removeAction
    }

    public var body: some View {
        avatar
            .overlay {
                if let removeAction {
                    removeButton(action: removeAction)
                        .offset(x: removeButtonOffset, y: -removeButtonOffset)
                }
            }
    }

    private var removeButtonOffset: CGFloat {
        size / 2 / CGFloat(2).squareRoot()
    }

    @ViewBuilder
    private var avatar: some View {
        if let action {
            Button(action: action) {
                image
            }
        } else {
            image
        }
    }

    private var image: some View {
        AssetImageView(
            assetImage: avatarImage,
            size: size,
        )
    }

    private func removeButton(action: @escaping () -> Void) -> some View {
        Button(action: action) {
            Images.System.xmark
                .resizable()
                .fontWeight(.semibold)
                .frame(size: .space12)
                .foregroundStyle(Colors.black)
                .padding(.small)
                .liquidGlass(in: Circle()) { view in
                    view
                        .background(Colors.listStyleColor)
                        .clipShape(Circle())
                }
                .padding(.small)
                .contentShape(Circle())
        }
        .buttonStyle(.borderless)
    }
}
