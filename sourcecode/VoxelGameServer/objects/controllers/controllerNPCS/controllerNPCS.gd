extends Node

var npcInstances = [];
var npcBase = preload("res://objects/npcs/npc_base/npc_base.tscn");

func _process(delta):
	if (Input.is_action_just_pressed("ui_accept")):
		var tmp = npcBase.instance();
		get_tree().current_scene.add_child(tmp);
		tmp.global_transform.origin = Vector3(0, 100, 0);

func AddNPC(npcObject:Spatial):
	npcInstances.append(npcObject);

func RemoveNPCWithID(npcID:int):
	var index = npcInstances.find(npcID);
	if (index == -1):
		push_error("NPC not found: " + str(npcID));
		return;
	RemoveNPCWithIndex(index);

func RemoveNPCWithIndex(index:int):
	#if (!is_instance_valid(npcInstances[index])):
	#	push_error("NPC index is invalid: " + str(index));
	#	return;
	instance_from_id(npcInstances[index]).queue_free();
	npcInstances.pop_at(index)
