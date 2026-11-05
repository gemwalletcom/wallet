// Copyright (c). Gem Wallet. All rights reserved.

import Components
import Primitives
import Style
import SwiftUI

struct ReceiveQRCodeCard: View {
    static var size: CGFloat {
        UIDevice.current.userInterfaceIdiom == .pad ? 180 : 260
    }

    private let image: UIImage?
    private let text: String
    private let action: () -> Void

    init(image: UIImage?, text: String, action: @escaping () -> Void) {
        self.image = image
        self.text = text
        self.action = action
    }

    var body: some View {
        Button(action: action) {
            VStack(spacing: .medium) {
                VStack {
                    if let image {
                        Image(uiImage: image)
                            .resizable()
                            .scaledToFit()
                            .padding(.extraSmall)
                            .background(Color.white)
                            .clipShape(RoundedRectangle(cornerRadius: .small))
                    } else {
                        LoadingView()
                    }
                }
                .frame(size: Self.size)

                Text(text.preventingHyphenation)
                    .multilineTextAlignment(.center)
                    .textStyle(TextStyle(font: .subheadline, color: Colors.secondaryText, fontWeight: .medium))
                    .fixedSize(horizontal: false, vertical: true)
                    .frame(maxWidth: Self.size)
                    .accessibilityIdentifier(text)
            }
        }
        .buttonStyle(.scale)
        .padding(.medium)
        .background(
            RoundedRectangle(cornerRadius: .medium)
                .fill(Colors.listStyleColor)
                .shadow(color: Color.black.opacity(.opacity25), radius: Sizing.shadow.radius, x: .zero, y: Sizing.shadow.yOffset),
        )
    }
}
