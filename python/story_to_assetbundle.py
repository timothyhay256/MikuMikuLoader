# story_to_assetbundle.py

# TODO: Cleanup

import os
import UnityPy
import UnityPy.config
import humps

import json

UnityPy.config.FALLBACK_UNITY_VERSION = "2022.3.21f1"

story_asset_path = ""
target_path_id = ""
target_object = None
env = None

def set_asset_path(asset_path):
    global story_asset_path

    print("Setting story asset path to "+str(asset_path))
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

# Returns the current typetree
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

    return typetree

def save_typetree(typetree):
    global target_object, env

    if target_object is None:
        print("Target object was not set. Something has gone wrong!")
        return

    target_object.save_typetree(typetree)

    with open(story_asset_path, "wb") as f:
        f.write(env.file.save())

    return "Success"