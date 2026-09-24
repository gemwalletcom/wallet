// Copyright (c). Gem Wallet. All rights reserved.

import Components
import enum Gemstone.GemContactAvatarImage
import Primitives
import Style

public extension GemContactAvatarImage {
    var assetImage: AssetImage {
        switch self {
        case let .initials(text): AssetImage(type: .text(text))
        case .placeholder: .image(Images.System.personCircleFill)
        case let .image(imageUrl, initials): AssetImage(type: .text(initials), imageURL: ImageSource(imageUrl).url)
        case let .emoji(emoji): AssetImage(type: .emoji(emoji))
        }
    }

    var style: AssetImageView.Style? {
        switch self {
        case .placeholder: AssetImageView.Style(foregroundColor: Colors.grayLightFaded)
        case .initials, .image, .emoji: nil
        }
    }
}
