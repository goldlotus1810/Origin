import subprocess

# 2 đoạn văn — không label
text_A = """Giao tranh ác liệt bùng phát tại khu vực phía đông khi các lực lượng 
vũ trang tiến hành tấn công vào tờ mờ sáng. Hàng chục dân thường thiệt mạng, 
hàng trăm người phải rời bỏ nhà cửa chạy loạn trong đêm tối. Bệnh viện dã chiến 
quá tải, không đủ thuốc men và nhân lực. Tiếng súng vẫn không ngừng vang lên 
suốt nhiều giờ đồng hồ. Trẻ em khóc, người già không thể chạy thoát, những ngôi 
nhà bốc cháy trong bóng đêm."""

text_B = """Scarlett O'Hara không đẹp, nhưng đàn ông ít khi nhận ra điều đó khi 
bị cuốn vào vẻ quyến rũ của cô. Đôi mắt xanh ngọc bích lấp lánh trong khuôn mặt 
thanh tú của cô gái xứ Georgia đã khiến nhiều trái tim rung động. Nhưng chiến tranh 
đã cướp đi tất cả — Tara hoang tàn, người thân ra đi, Ashley đã thuộc về người khác. 
Cô đứng trên mảnh đất đỏ của Tara, bàn tay nắm chặt một củ cải đất, thề rằng 
dù trời có sập cô cũng sẽ không bao giờ đói nữa."""

texts = [("Đoạn A", text_A), ("Đoạn B", text_B)]

for label, text in texts:
    print(f"\n{'='*50}")
    print(f"{label}:")
    print(f"{'='*50}")
    print(text[:80] + "...")
    print()

print("\n" + "="*50)
print("Chạy BookReader + LearningLoop trên cả 2 đoạn...")
print("="*50)
