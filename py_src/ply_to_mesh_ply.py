# coding: utf-8

# 1.Rustで作成したplzファイルをPymeshlabでロード
# 2.メッシュを作成
# 3.open3dデータに変換 ⇒ やり方が分からないので中間ファイルを生成(with_mesh.plz)
# 4.視点設定を行い画像を作成 ⇒ カメラの設定方法について追究する必要がある
# 5.pngとして保存


import pymeshlab
import sys


def main():
    # pcdをplz に変換して保存
    input_filepath = sys.argv[1]
    output_filepath =sys.argv[2]

    # 1.Rustで作成したplzファイルをPymeshlabでロード
    ms = pymeshlab.MeshSet()
    ms.load_new_mesh(input_filepath)

    # 2.メッシュを作成
    ms.apply_filter("compute_normals_for_point_sets")
    ms.apply_filter("surface_reconstruction_screened_poisson")
    # ms.apply_filter('remove_isolated_pieces_wrt_diameter', mincomponentdiag=50)
    ms.apply_filter("remove_isolated_pieces_wrt_diameter")

    # 3.open3dデータに変換
    
    ms.save_current_mesh(output_filepath)



if __name__ == "__main__":
    if len(sys.argv) != 3:
        print("引数が正しくない")
        quit()
    main()
    # print(sys.argv[1],sys.argv[2],sys.argv[3])