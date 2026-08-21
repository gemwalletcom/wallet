// Copyright (c). Gem Wallet. All rights reserved.

import UIKit

public enum EmojiAvatarRenderer {
    @MainActor
    public static func image(emoji: String, size: CGFloat, color: UIColor) -> UIImage {
        let format = UIGraphicsImageRendererFormat()
        format.scale = UIScreen.main.scale

        let renderer = UIGraphicsImageRenderer(
            size: CGSize(width: size, height: size),
            format: format,
        )

        return renderer.image { context in
            let rect = CGRect(x: 0, y: 0, width: size, height: size)

            let path = UIBezierPath(ovalIn: rect.insetBy(dx: -0.5, dy: -0.5))
            UIColor.clear.setFill()
            context.fill(rect)

            color.setFill()
            path.fill()

            let font = UIFont.boldSystemFont(ofSize: size * AvatarScale.emoji)
            let paragraphStyle = NSMutableParagraphStyle()
            paragraphStyle.alignment = .center

            let attributes: [NSAttributedString.Key: Any] = [
                .font: font,
                .paragraphStyle: paragraphStyle,
            ]
            let attributedString = NSAttributedString(string: emoji, attributes: attributes)

            let textSize = attributedString.size()
            attributedString.draw(in: CGRect(
                x: (size - textSize.width) / 2,
                y: (size - textSize.height) / 2,
                width: textSize.width,
                height: textSize.height,
            ))
        }
    }
}
