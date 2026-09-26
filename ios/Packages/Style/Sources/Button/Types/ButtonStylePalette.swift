// Copyright (c). Gem Wallet. All rights reserved.

import SwiftUI

public struct ButtonStylePalette: Hashable, Sendable {
    public let foreground: Color
    public let foregroundPressed: Color
    public let background: Color
    public let backgroundPressed: Color
    public let backgroundDisabled: Color

    public init(
        foreground: Color,
        foregroundPressed: Color,
        background: Color,
        backgroundPressed: Color,
        backgroundDisabled: Color,
    ) {
        self.foreground = foreground
        self.foregroundPressed = foregroundPressed
        self.background = background
        self.backgroundPressed = backgroundPressed
        self.backgroundDisabled = backgroundDisabled
    }
}

// MARK: - Static

extension ButtonStylePalette {
    static let primary = ButtonStylePalette(
        foreground: Colors.whiteSolid,
        foregroundPressed: Colors.whiteSolid,
        background: Colors.blue,
        backgroundPressed: Colors.blueDark,
        backgroundDisabled: Colors.blueDarkFaded,
    )

    static let blue = ButtonStylePalette(
        foreground: Colors.whiteSolid,
        foregroundPressed: Colors.whiteSolid,
        background: Colors.blue,
        backgroundPressed: Colors.blueDark,
        backgroundDisabled: Colors.blueFaded,
    )

    static let blueGrayPressed = ButtonStylePalette(
        foreground: Colors.whiteSolid,
        foregroundPressed: Colors.whiteSolid,
        background: Colors.blue,
        backgroundPressed: Colors.gray,
        backgroundDisabled: Colors.blueFaded,
    )

    static let lightGray = ButtonStylePalette(
        foreground: Colors.gray,
        foregroundPressed: Colors.whiteSolid,
        background: Colors.grayVeryLight,
        backgroundPressed: Colors.grayLightFaded,
        backgroundDisabled: Colors.grayVeryLightFaded,
    )

    static let empty = ButtonStylePalette(
        foreground: Colors.black,
        foregroundPressed: Colors.blackFaded,
        background: Colors.Empty.buttonsBackground,
        backgroundPressed: Colors.Empty.buttonsBackground,
        backgroundDisabled: Colors.Empty.buttonsBackground,
    )

    static let amount = ButtonStylePalette(
        foreground: Colors.black,
        foregroundPressed: Colors.blackFaded,
        background: Colors.Empty.listEmpty,
        backgroundPressed: Colors.grayVeryLightFaded,
        backgroundDisabled: Colors.grayVeryLightFaded,
    )

    static let listStyleColor = ButtonStylePalette(
        foreground: Colors.gray,
        foregroundPressed: Colors.whiteSolid,
        background: Colors.listStyleColor,
        backgroundPressed: Colors.grayVeryLight,
        backgroundDisabled: Colors.grayFaded,
    )

    static let red = ButtonStylePalette(
        foreground: Colors.whiteSolid,
        foregroundPressed: Colors.whiteSolid,
        background: Colors.red,
        backgroundPressed: Colors.redFaded,
        backgroundDisabled: Colors.redFadedLight,
    )

    static let green = ButtonStylePalette(
        foreground: Colors.whiteSolid,
        foregroundPressed: Colors.whiteSolid,
        background: Colors.green,
        backgroundPressed: Colors.greenFaded,
        backgroundDisabled: Colors.greenFadedLight,
    )

    static let listEmpty = ButtonStylePalette(
        foreground: Colors.gray,
        foregroundPressed: Colors.black,
        background: Colors.Empty.listEmpty,
        backgroundPressed: Colors.Empty.listEmpty,
        backgroundDisabled: Colors.Empty.listEmpty,
    )
}
