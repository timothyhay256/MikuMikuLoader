# story_to_assetbundle.py

# TODO: Cleanup

import os
import UnityPy
import UnityPy.config

from PIL import Image
import io

import json

UnityPy.config.FALLBACK_UNITY_VERSION = "2022.3.21f1"

story_asset_path = ""
target_path_id = ""
target_object = None
env = None

def set_asset_path(asset_path):
    global story_asset_path

    print("Setting asset path to "+str(asset_path))
    story_asset_path = asset_path

def set_target_path_id(path_id):
    global target_path_id

    print("Setting target path id to "+str(path_id))
    target_path_id = path_id

def print_typetree(data, indent=0):
    prefix = " " * indent
    if isinstance(data, dict):
        for key, value in data.items():
            print(f"{prefix}{key}:")
            print_typetree(value, indent + 2)
    elif isinstance(data, list):
        for i, item in enumerate(data):
            print(f"{prefix}[{i}]:")
            print_typetree(item, indent + 2)
    else:
        print(f"{prefix}{repr(data)}")

def list_assets():
    global story_asset_path

    env = UnityPy.load(story_asset_path)
    for obj in env.objects:
        print(f"\nName: [{obj.type.name}] PathID: {obj.path_id}")
        try:
            name = "<unknown>"
            try:
                instance = obj.read()
                name = getattr(instance, "name", "<no name>")
            except:
                pass

            print(f"Name: {name}")
            print("Typetree:")
            typetree = obj.read_typetree()
            print_typetree(typetree)
        except Exception as e:
            print(f"Could not read object: {e}")

# Returns the current typetree TODO: Use name instead of path_id
def return_typetree():
    global story_asset_path, target_path_id, target_object, env

    env = UnityPy.load(story_asset_path)

    for obj in env.objects:
        if obj.path_id == target_path_id:
            target_object = obj
            break

    if target_object is None:
        print(f"No object found with path_id {target_path_id}")
        return

    if target_object.type.name != "MonoBehaviour":
        print("Object is not an MonoBehaviour.")

    typetree = target_object.read_typetree()

    # print(typetree)
    # print(json.dumps(typetree, sort_keys=True, indent=4))
    # list_assets()

    return typetree

def return_texture2d_img(object_name):
    global story_asset_path, target_path_id, target_object, env

    env = UnityPy.load(story_asset_path)

    print("Obtaining correct asset object")
    if target_object is None or target_object.read().m_Name != object_name:
        # Cache the object after the first lookup
        for obj in env.objects:
            data = obj.read()
            if data.m_Name == object_name and obj.type.name == "Texture2D":
                target_object = obj
                break
        else:
            print(f"No object found with name {object_name}")
            return

    if target_object.type.name != "Texture2D":
        print("Object is not an Texture2D.")

    data = target_object.read()

    img_byte_arr = io.BytesIO()
    data.image.save(img_byte_arr, format='PNG')

    return img_byte_arr.getvalue()

#TODO Cleanup/make more efficient
def save_texture2d_img(object_name, new_image_path, write_changes):
    global target_object, env, story_asset_path

    if env == None:
        env = UnityPy.load(story_asset_path)

    print("Obtaining correct asset object")
    if target_object is None or target_object.read().m_Name != object_name:
        # Cache the object after the first lookup
        for obj in env.objects:
            data = obj.read()
            if data.m_Name == object_name and obj.type.name == "Texture2D":
                target_object = obj
                break
        else:
            print(f"No object found with name {object_name}")
            return

    print("Opening image")
    image = Image.open(new_image_path)
    print("Saving image")
    data.image = image
    data.save()

    if write_changes:
        print("Writing changes")
        with open(story_asset_path, "wb") as f:
            f.write(env.file.save())

#TODO Cleanup/make more efficient
def save_sprite_img(object_name, new_image_path, write_changes):
    global target_object, env, story_asset_path

    if env == None:
        env = UnityPy.load(story_asset_path)

    print("Obtaining correct asset object")
    for obj in env.objects:
      if target_object is None or target_object.read().m_Name != object_name:
        # Cache the object after the first lookup
        for obj in env.objects:
            data = obj.read()
            if data.m_Name == object_name and obj.type.name == "Sprite":
                target_object = obj
                break
        else:
            print(f"No object found with name {object_name}")
            return

    data = target_object.read()
    
    print("Saving new logo into sprite")
    data.image.save(new_image_path)

    # data.save()

    if write_changes:
        print("Writing changes")
        with open(story_asset_path, "wb") as f:
            f.write(env.file.save())


def save_typetree(typetree):
    global target_object, env

    if target_object is None:
        print("Target object was not set. Something has gone wrong!")
        return

    target_object.save_typetree(typetree)

    with open(story_asset_path, "wb") as f:
        f.write(env.file.save())

    return "Success"