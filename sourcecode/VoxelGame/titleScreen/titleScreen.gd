extends Control

onready var server_ip: LineEdit = (
	$CenterContainer/VBoxContainer/MultiplayerVBC/HBoxContainer/ServerIP
)

func _ready():
	get_tree().get_root().set_meta("world_name", null)

func _on_ConnectButton_pressed():
	get_tree().get_root().set_meta("server_ip", server_ip.text)
	get_tree().change_scene("res://objects/game/Game.tscn")

func _on_ServerIP_text_entered(_new_text):
	_on_ConnectButton_pressed()

func _on_Worlds_world_chosen(world_name):
	get_tree().get_root().set_meta("world_name", world_name)
	get_tree().change_scene("res://objects/game/Game.tscn")
