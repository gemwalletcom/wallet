import SwiftUI

public enum Colors {
    public static let white = Color.dynamicColor("#FFFFFF", dark: "#222222")
    public static let whiteSolid = Color.dynamicColor("FFFFFF")
    public static let black = Color.dynamicColor("#222222", dark: "FFFFFF")
    public static let blue = Color.dynamicColor("#2D5BE6")
    public static let blueDark = Color.dynamicColor("#1742C5")
    public static let red = Color.dynamicColor("#F84E4E")
    public static let redLight = Color.dynamicColor("#FFF1F1", dark: "#462D30")
    public static let green = Color.dynamicColor("#1B9A6C")
    public static let greenLight = Color.dynamicColor("EAFAF5", dark: "27423C")
    public static let orange = Color.dynamicColor("#FF9314")
    public static let gray = Color.dynamicColor("#818181")
    public static let grayLight = Color.dynamicColor("#969996")
    public static let grayVeryLight = Color.dynamicColor("#F4F4F4", dark: "#333333")
    public static let grayBackground = Color.dynamicColor("#F2F2F7", dark: "#1C1C1E")
    public static let grayDarkBackground = Color.dynamicColor("#E6E6F0", dark: "#1C1C1E")
    public static let secondaryText = Color.dynamicColor("#818181")
    public static let listStyleColor = UIColor.dynamicColor(UIColor.systemBackground.color, dark: UIColor.secondarySystemBackground.color)
    public static let insetGroupedListStyle = UIColor.dynamicColor(UIColor.systemGroupedBackground.color, dark: UIColor.black.color)
    public static let sheetInsetGroupedListStyle = Color(uiColor: .systemGroupedBackground)
}

// MARK: - Empty

public extension Colors {
    enum Empty {
        public static let imageBackground = Color(.quaternaryLabel)
        public static let image = Color.dynamicColor("#767A81")
        public static let buttonsBackground = Color(.quaternaryLabel)
        public static let listEmpty = Color(.secondarySystemFill)
    }
}

// MARK: - Faded Variants

public extension Colors {
    static let blueFaded = Color.dynamicColor("#6BA3F1", dark: "#5A96ED")
    static let blueDarkFaded = Color.dynamicColor("#6085E9", dark: "#5A7CE6")
    static let whiteFaded = Color.dynamicColor("#F0F0F0", dark: "#444444")
    static let grayFaded = Color.dynamicColor("#C0C0C0", dark: "#606060")
    static let grayLightFaded = Color.dynamicColor("#CACBCA", dark: "#4D524D")
    static let grayVeryLightFaded = Color.dynamicColor("#F9F9F9", dark: "#666666")
    static let blackFaded = Color.dynamicColor("#919191", dark: "#7F7F7F")
    static let redFaded = Color.dynamicColor("#FB7676", dark: "#FB7676")
    static let redFadedLight = Color.dynamicColor("#FC8E8E", dark: "#FC8E8E")
    static let greenFaded = Color.dynamicColor("#4EAC84", dark: "#4EAC84")
    static let greenFadedLight = Color.dynamicColor("#67B593", dark: "#67B593")
}

#Preview {
    let colors: [(name: String, color: Color)] = [
        ("White", Colors.white),
        ("White Solid", Colors.whiteSolid),
        ("Black", Colors.black),
        ("Blue", Colors.blue),
        ("Blue Dark", Colors.blueDark),
        ("Red", Colors.red),
        ("Red Light", Colors.redLight),
        ("Green", Colors.green),
        ("Green Light", Colors.greenLight),
        ("Orange", Colors.orange),
        ("Gray", Colors.gray),
        ("Gray Light", Colors.grayLight),
        ("Gray Very Light", Colors.grayVeryLight),
        ("Gray Background", Colors.grayBackground),
        ("Gray Dark Background", Colors.grayDarkBackground),
        ("Secondary Text", Colors.secondaryText),
        ("List style color", Colors.listStyleColor),
        ("Inset Gropued List style color", Colors.insetGroupedListStyle),
    ]
    return List {
        ForEach(colors, id: \.name) { color in
            HStack {
                Text(color.name)
                    .multilineTextAlignment(.leading)
                    .frame(width: 174)

                RoundedRectangle(cornerRadius: 4)
                    .fill(color.color)
                    .padding(.extraSmall)
                    .colorScheme(.light)

                RoundedRectangle(cornerRadius: 4)
                    .fill(color.color)
                    .padding(.extraSmall)
                    .colorScheme(.dark)
            }
            .listRowInsets(EdgeInsets(top: 0, leading: 8, bottom: 0, trailing: 0))
        }
    }
    .listStyle(InsetGroupedListStyle())
}
