from telethon import TelegramClient
from telethon.tl.functions.channels import GetForumTopicsByIDRequest
import os, sys

api_id = sys.argv[1]
api_hash = sys.argv[2]
client = TelegramClient('session_name', api_id, api_hash)
channel_id = int(sys.argv[3])
topic_id = int(sys.argv[4])
file_dir = sys.argv[5]

async def main():
    await client.start()

    '''
    topics = await client(GetForumTopicsRequest(
        channel=channel_id,
        offset_date=0, #datetime(2025, 3, 19),
        offset_id=0,
        offset_topic=0,
        limit=100
        #q='some string here'
    ))
    print(topics.count, "topics.")
    for topic in topics.topics:
        print(topic.id, "::", topic.title)
    '''

    # get users
    users = await client.get_participants(channel_id, aggressive=True)
    # get topic - exist if err
    topic = None
    topics = await client(GetForumTopicsByIDRequest(
        channel=channel_id,
        topics=[topic_id]
    ))
    if topics.count != 1:
        print("Topic {} not found.".format(topic_id))
        return
    topic = topics.topics[0]
    # log file prep
    file_name = "{}_{}".format(topic_id, topic.title.replace(" ", "_").replace(",", "").lower())
    file_name = file_name[:42]
    file_path = "{}/{}.log".format(file_dir, file_name)
    # file output lines
    file_lines = []
    file_lines.append(topic.title)
    file_lines.append("")
    # traverse messages in topic
    async for message in client.iter_messages(channel_id, reply_to=topic_id, limit=5000, reverse=True):
        # print(json.dumps(message.to_dict(), indent=4, sort_keys=True, default=str))
        message_user_id = message.from_id.user_id
        message_user = None
        for user in users:
            if user.id == message_user_id:
                message_user = user
                break
        if message.message != None:
            file_lines.append(message.date)
            if message_user != None:
                name_display = []
                if message_user.first_name:
                    name_display.append(message_user.first_name)
                if message_user.last_name:
                    name_display.append(message_user.last_name)
                if message_user.username:
                    name_display.append("(@{})".format(message_user.username))
                file_lines.append(" ".join(name_display))
            else:
                file_lines.append("Deleted User @{}".format(message_user_id))
            if message.media:
                file_lines.append("[media]")
            if len(message.message) > 0:
                file_lines.append(message.message)
            file_lines.append("")
        if os.path.exists(file_path):
            os.remove(file_path)
        with open(file_path, 'w') as f:
            for line in file_lines:
                f.write(f"{line}\n")
    print(file_path)

client.loop.run_until_complete(main())
